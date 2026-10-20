import { Session } from './session.js';
import { mount } from './ui.js';

const session = new Session({ bluetooth: navigator.bluetooth, protocol: location.protocol });
mount(session);
setInterval(() => session.tick(), 25);
fetch('./version.txt').then(async response => {
  if (response.ok) document.getElementById('version').textContent = `Build ${(await response.text()).trim()}`;
}).catch(() => {}); // Source previews have no generated revision file.
