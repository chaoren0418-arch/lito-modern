// Output formatting only. Raw complete UIA observations remain in Sky-events.json.
globalThis.flowSummaryCache = new Map();
globalThis.flowSummary = state => {
  const controls = (state.accessibility?.tree || '').split('\n').filter(line =>
    !line.startsWith('The focused UI element') &&
    (/ID: (10[1-9]|11[0-9]|12[01]|20[1-9]|21[0-6])\b/.test(line) || line.includes('清單項目')));
  const current = JSON.stringify(controls);
  const unchanged = flowSummaryCache.get(state.window.id) === current;
  flowSummaryCache.set(state.window.id, current);
  return JSON.stringify({focus: state.accessibility?.focused_element ?? null,
    ...(unchanged ? {listed_controls_unchanged: true} : {controls})});
};
