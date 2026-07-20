import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'
import Probe from './lib/Probe.svelte'

const target = document.getElementById('app')
if (!target) {
  throw new Error('missing #app mount point')
}

// #probe swaps in the throwaway design-system verification surface (ticket 01);
// the cockpit is otherwise unchanged. Removed with Probe.svelte in tickets 02–04.
const Root = location.hash === '#probe' ? Probe : App

export default mount(Root, { target })
