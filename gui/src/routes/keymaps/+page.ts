import type { PageData } from './$types'
import type { Keymap } from 'src/models/keymap'

const mockKeymaps: Keymap[] = [
  {
    type: 'harmonic',
    id: '1',
    name: 'C Major',
    scaleTones: new Set([
      0, 2, 3, 4, 
    ])
  }
]

export async function load({ params }: { params: PageData }) {
  console.log('load params', params)

}