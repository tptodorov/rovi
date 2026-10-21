import { Session } from './session.8fb8fd174d30.js';
import { mount } from './ui.8fb8fd174d30.js';

const session = new Session({ bluetooth: navigator.bluetooth, protocol: location.protocol });
mount(session);
setInterval(() => session.tick(), 25);
const revision = document.querySelector('meta[name="rovi-build"]')?.content;
if (revision && revision !== 'source-checkout') document.getElementById('version').textContent = `Build ${revision}`;
