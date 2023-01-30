import type { PitchClass, Tuning } from "./tuning"

export type KeymapBase = {
  type: 'harmonic' | 'freeform',
  id: string,
  name: string,
}


export type InactiveKeyBehavior 
  = 'midi-off' 
  | 'light-dim'

export type HarmonicKeymap = KeymapBase & {
  type: 'harmonic',

  /**
   * What tuning is this keymap in?
   * Defaults to 12EDO if not present.
   */
  tuning?: Tuning,

  /** 
   * What to do with non-scale notes?
   * Defaults to ['light-dim'] if not present.
   */
  nonScaleToneBehaviors?: InactiveKeyBehavior[],

  // set of enabled pitches
  scalePitches: Set<PitchClass>,
}

export type FreeformKeymap = KeymapBase & {
  type: 'freeform',


}

export type Keymap 
  = HarmonicKeymap 
  | FreeformKeymap
