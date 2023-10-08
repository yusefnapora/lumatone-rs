use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;
use midir::{MidiInput, MidiOutput};
use log::{debug, info, warn};
use error_stack::{report, IntoReport, Result, ResultExt};

use crate::{
  commands::ping,
  device::LumatoneDevice,
  error::LumatoneMidiError,
  responses::decode_ping,
};

const CLIENT_NAME: &'static str = "lumatone_rs";

pub async fn detect_device() -> Result<LumatoneDevice, LumatoneMidiError> {
  use LumatoneMidiError::DeviceDetectionFailed;
  debug!("beginning lumatone device detection");

  let output = MidiOutput::new(CLIENT_NAME)
    .into_report()
    .change_context(DeviceDetectionFailed)?;

  let input = MidiInput::new(CLIENT_NAME)
    .into_report()
    .change_context(DeviceDetectionFailed)?;
  let in_ports = input.ports();
  let out_ports = output.ports();

  debug!(
    "found {} input ports and {} output ports",
    in_ports.len(),
    out_ports.len()
  );

  let (tx, mut rx) = mpsc::channel(in_ports.len());

  let mut input_connections = vec![];
  for (port_index, p) in in_ports.iter().enumerate() {
    // unfortunately, it doesn't seem to be possible to use the same MidiInput to connect to
    // multiple ports in parallel, since MidiInput.connect consumes self.
    let midi_in = MidiInput::new(CLIENT_NAME)
      .into_report()
      .change_context(DeviceDetectionFailed)?;
    let port_name = midi_in
      .port_name(p)
      .into_report()
      .change_context(DeviceDetectionFailed)?;
    let my_tx = tx.clone();
    let conn_res = midi_in.connect(
      p,
      &port_name,
      move |_, msg, _| {
        match decode_ping(msg) {
          Ok(output_port_index) => {
            let _ = my_tx.blocking_send((port_index, output_port_index as usize));
            // TODO: don't swallow channel send errors
          }
          Err(e) => {
            warn!("error decoding ping message: {:?}", e);
          }
        }
      },
      (),
    );
    match conn_res {
      Ok(conn) => {
        info!("connected to input port {port_name}");
        input_connections.push(conn);
      }
      Err(e) => warn!("input connection error for port {port_name}: {e}"),
    }
  }

  // send a ping message on all output ports, with the ping value set to the output port index
  for (port_index, p) in out_ports.iter().enumerate() {
    let midi_out = MidiOutput::new(CLIENT_NAME)
      .into_report()
      .change_context(DeviceDetectionFailed)?;
    let port_name = midi_out
      .port_name(p)
      .into_report()
      .change_context(DeviceDetectionFailed)?;
    if let Ok(mut conn) = midi_out.connect(p, &port_name) {
      let cmd = ping(port_index as u32);
      if let Err(send_err) = conn.send(&cmd.to_sysex_message()) {
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
    return Err(report!(LumatoneMidiError::DeviceDetectionFailed).attach_printable("timed out"));
  }

  let output_port_name = output
    .port_name(&out_ports[out_port_idx.unwrap()])
    .into_report()
    .change_context(DeviceDetectionFailed)?;
  let input_port_name = input
    .port_name(&in_ports[in_port_idx.unwrap()])
    .into_report()
    .change_context(DeviceDetectionFailed)?;

  info!("detected lumatone ports: in: {input_port_name}, out: {output_port_name}");

  let device = LumatoneDevice::new(&output_port_name, &input_port_name);
  Ok(device)
}
