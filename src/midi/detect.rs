use std::{error::Error, time::Duration};
use tokio::sync::mpsc;
use tokio::time::timeout;

use super::{device::LumatoneDevice, commands::{ping, decode_ping}};
use midir::{MidiOutput, MidiInput};

use log::{debug, warn, info};

const CLIENT_NAME: &'static str = "lumatone_rs";

pub async fn detect_device() -> Result<LumatoneDevice, Box<dyn Error>> {
  debug!("beginning lumatone device detection");

  let output = MidiOutput::new(CLIENT_NAME)?;
  let input = MidiInput::new(CLIENT_NAME)?;
  let in_ports = input.ports();
  let out_ports = output.ports();

  debug!("found {} input ports and {} output ports", in_ports.len(), out_ports.len());

  let (tx, mut rx) = mpsc::channel(in_ports.len());

  let mut input_connections = vec![];
  for (port_index, p) in in_ports.iter().enumerate() {
    // unfortunately, it doesn't seem to be possible to use the same MidiInput to connect to
    // multiple ports in parallel, since MidiInput.connect consumes self.
    let midi_in = MidiInput::new(CLIENT_NAME)?;
    let port_name = midi_in.port_name(p)?;
    let my_tx = tx.clone();
    let conn_res = midi_in.connect(p, &port_name, move |_, msg, _| {

      match decode_ping(msg) {
        Ok(output_port_index) => {
          let _ = my_tx.blocking_send((port_index, output_port_index as usize)); // TODO: don't swallow channel send errors
        },
        Err(e) => {
          warn!("error decoding ping message: {:?}", e);
        }
    }
    }, ());
    match conn_res { 
      Ok(conn) => {
        info!("connected to input port {port_name}");
        input_connections.push(conn);
      },
      Err(e) => warn!("input connection error for port {port_name}: {e}")
    }
  }

  // send a ping message on all output ports, with the ping value set to the output port index
  for (port_index, p) in out_ports.iter().enumerate() {
    let midi_out = MidiOutput::new(CLIENT_NAME)?;
    let port_name = midi_out.port_name(p)?;
    if let Ok(mut conn) = midi_out.connect(p, &port_name) {
      let msg = ping(port_index as u32);
      if let Err(send_err) = conn.send(&msg) {
        warn!("send error: {send_err}");
      }
      debug!("sent ping on output {port_index} - {port_name}");
      conn.close();
    }
  }

  let mut in_port_idx: Option<usize> = None;
  let mut out_port_idx: Option<usize> = None;
  let with_timeout = timeout(Duration::from_secs(30), rx.recv());
  while let Ok(Some((in_port_index, out_port_index))) = with_timeout.await {
    in_port_idx = Some(in_port_index);
    out_port_idx = Some(out_port_index);
    break;
  }

  if in_port_idx.is_none() || out_port_idx.is_none() {
    return Err("unable to detect lumatone ports".into())
  }

  let output_port_name = output.port_name(&out_ports[out_port_idx.unwrap()])?;
  let input_port_name = input.port_name(&in_ports[in_port_idx.unwrap()])?;

  info!("detected lumatone ports: in: {input_port_name}, out: {output_port_name}");

  let device =  LumatoneDevice::new(&output_port_name, &input_port_name);
  Ok(device)
}
