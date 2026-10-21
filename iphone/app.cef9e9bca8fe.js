import { Session } from './session.cef9e9bca8fe.js';
import { mount } from './ui.cef9e9bca8fe.js';

const session = new Session({ bluetooth: navigator.bluetooth, protocol: location.protocol });
mount(session);
setInterval(() => session.tick(), 25);
const revision = document.querySelector('meta[name="rovi-build"]')?.content;
if (revision && revision !== 'source-checkout') document.getElementById('version').textContent = `Build ${revision}`;
