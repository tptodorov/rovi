const requirements = [...document.querySelectorAll('.requirement')];
const search = document.querySelector('#search');
function filter() {
  const query = search.value.trim().toLocaleLowerCase();
  for (const item of requirements) item.hidden = !item.textContent.toLocaleLowerCase().includes(query);
  const count = requirements.filter(item => !item.hidden).length;
  document.querySelector('#matches').textContent = `${count} of ${requirements.length} requirements shown`;
  document.querySelector('#empty').hidden = count !== 0;
}
search.addEventListener('input', filter);
document.querySelector('#clear').onclick = () => { search.value = ''; filter(); search.focus(); };
for (const [id, open] of [['expand', true], ['collapse', false]]) {
  document.querySelector('#' + id).onclick = () => {
    for (const item of requirements.filter(item => !item.hidden)) {
      for (const scenario of item.querySelectorAll('details')) scenario.open = open;
    }
  };
}
function revealTarget() {
  let target;
  try { target = document.getElementById(decodeURIComponent(location.hash.slice(1))); } catch { return; }
  if (!target) return;
  search.value = ''; filter();
  if (target.matches('details')) target.open = true;
  for (let parent = target.parentElement; parent; parent = parent.parentElement) {
    if (parent.matches('details')) parent.open = true;
  }
  target.scrollIntoView();
}
window.addEventListener('hashchange', revealTarget);
let printState;
window.addEventListener('beforeprint', () => {
  printState = [...document.querySelectorAll('details,.requirement')].map(el => [el, el.open, el.hidden]);
  for (const [el] of printState) { el.hidden = false; if (el.matches('details')) el.open = true; }
});
window.addEventListener('afterprint', () => {
  for (const [el, open, hidden] of printState ?? []) { el.hidden = hidden; if (el.matches('details')) el.open = open; }
});
document.querySelector('#print').onclick = () => window.print();
filter();
if (location.hash) revealTarget();
