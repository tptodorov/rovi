import { Session } from './session.js';
import { mount } from './ui.js';

const session = new Session({ bluetooth: navigator.bluetooth, protocol: location.protocol });
mount(session);
setInterval(() => session.tick(), 25);
const revision = document.querySelector('meta[name="rovi-build"]')?.content;
if (revision && revision !== 'source-checkout') document.getElementById('version').textContent = `Build ${revision}`;
