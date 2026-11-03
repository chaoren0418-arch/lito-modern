// Future GUI QA helpers. Importing performs no desktop input or observation.
// Call only after an inspected, fresh flowObserve/flowAct result.
globalThis.flowRequireWorld = (role, map) => {
  const state = flowQA.states[role];
  if (!state?.accessibility) throw Error('Observe the target world before input');
  const tree = state.accessibility.tree || '';
  const focus = state.accessibility.focused_element || '';
  const status = tree.split('\n').find(line => /\bID: 204\b/.test(line)) || '';
  if (!Number.isInteger(map) || !status.includes('地圖 ' + map + ' ')
      || /正在載入|載入失敗|等待.*轉場/.test(status)
      || !/^0 視窗/.test(focus)) {
    throw Error('World map ' + map + ' is not observed ready with root focus; observe again, do not move');
  }
  return state;
};
globalThis.flowWorldAct = async (role, key, label, map) => {
  // Exactly one actual input and its immediate readback; no hidden retry/wait.
  flowRequireWorld(role, map);
  await flowAct(role, 'press_key', {key}, label);
};
globalThis.flowRequireQuantity = role => {
  const state = flowQA.states[role];
  if (!state?.accessibility) throw Error('Observe the current quantity field');
  const focus = state.accessibility.focused_element || '';
  if (!/\bID: 222\b/.test(focus) || /disabled/.test(focus)
      || !(state.accessibility.tree || '').includes('請輸入 1～8 的數量')) {
    throw Error('The received, enabled hotel quantity field is not focused');
  }
};
globalThis.flowQuantityEnter = async (role, label) => {
  flowRequireQuantity(role);
  await flowAct(role, 'press_key', {key: 'Return'}, label);
};
