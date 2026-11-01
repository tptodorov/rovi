import { Session } from './session.8e84bdaac9c2.js';
import { mount } from './ui.8e84bdaac9c2.js';
import { protocolError } from './wire.8e84bdaac9c2.js';

const session = new Session({ bluetooth: navigator.bluetooth, protocol: location.protocol });
if (protocolError()) session.message = protocolError();
mount(session);
setInterval(() => session.tick(), 25);
const revision = document.querySelector('meta[name="rovi-build"]')?.content;
if (revision && revision !== 'source-checkout') document.getElementById('version').textContent = `Build ${revision}`;
