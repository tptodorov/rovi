import { Session } from './session.712ef024b110.js';
import { mount } from './ui.712ef024b110.js';
import { protocolError } from './wire.712ef024b110.js';

const session = new Session({ bluetooth: navigator.bluetooth, protocol: location.protocol });
if (protocolError()) session.message = protocolError();
mount(session);
setInterval(() => session.tick(), 25);
const revision = document.querySelector('meta[name="rovi-build"]')?.content;
if (revision && revision !== 'source-checkout') document.getElementById('version').textContent = `Build ${revision}`;
