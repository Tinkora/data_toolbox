import init, { convert_csv, inspect_csv } from './pkg/data_toolbox_web.js';

const elements = {
  status: document.querySelector('#wasm-status'),
  runtime: document.querySelector('.runtime'),
  fatalError: document.querySelector('#fatal-error'),
  input: document.querySelector('#csv-input'),
  file: document.querySelector('#file-input'),
  clear: document.querySelector('#clear-button'),
  inspect: document.querySelector('#inspect-button'),
  delimiter: document.querySelector('#delimiter'),
  headers: document.querySelector('#headers'),
  resultState: document.querySelector('#result-state'),
  emptyState: document.querySelector('#empty-state'),
  resultContent: document.querySelector('#result-content'),
  rowCount: document.querySelector('#row-count'),
  columnCount: document.querySelector('#column-count'),
  selectedDelimiter: document.querySelector('#selected-delimiter'),
  diagnosticCount: document.querySelector('#diagnostic-count'),
  diagnostics: document.querySelector('#diagnostics'),
  previewHead: document.querySelector('#preview-table thead'),
  previewBody: document.querySelector('#preview-table tbody'),
  outputFormat: document.querySelector('#output-format'),
  formulaPolicy: document.querySelector('#formula-policy'),
  export: document.querySelector('#export-button'),
  exportOutput: document.querySelector('#export-output'),
};

let wasmReady = false;
let hasInspection = false;
const MAX_FILE_BYTES = 10 * 1024 * 1024;

function inspectOptions() {
  return JSON.stringify({
    delimiter: elements.delimiter.value,
    headers: elements.headers.value,
  });
}

function convertOptions() {
  return JSON.stringify({
    delimiter: elements.delimiter.value,
    headers: elements.headers.value,
    output: elements.outputFormat.value,
    formula_policy: elements.formulaPolicy.value,
  });
}

function updateActions() {
  const hasInput = elements.input.value.length > 0;
  elements.inspect.disabled = !(wasmReady && hasInput);
  elements.export.disabled = !(wasmReady && hasInput && hasInspection);
}

function invalidateInspection() {
  hasInspection = false;
  elements.exportOutput.value = '';
  elements.resultContent.hidden = true;
  elements.emptyState.hidden = false;
  elements.resultState.textContent = 'Waiting for inspection';
  delete elements.resultState.dataset.tone;
  clearError();
  updateActions();
}

function setError(error) {
  elements.fatalError.replaceChildren();
  const code = document.createElement('strong');
  code.textContent = error.code;
  const message = document.createElement('span');
  message.textContent = error.message;
  elements.fatalError.append(code, message);
  elements.fatalError.hidden = false;
  elements.resultState.textContent = 'Needs attention';
  elements.resultState.dataset.tone = 'danger';
}

function clearError() {
  elements.fatalError.hidden = true;
  elements.fatalError.replaceChildren();
}

function makeCell(tagName, text, scope) {
  const cell = document.createElement(tagName);
  cell.textContent = text;
  if (scope) cell.scope = scope;
  return cell;
}

function renderDiagnostics(diagnostics) {
  elements.diagnostics.replaceChildren();
  if (diagnostics.length === 0) {
    const item = document.createElement('li');
    item.className = 'diagnostic-clear';
    item.textContent = 'No structural diagnostics found.';
    elements.diagnostics.append(item);
    return;
  }

  for (const diagnostic of diagnostics) {
    const item = document.createElement('li');
    const heading = document.createElement('div');
    heading.className = 'diagnostic-heading';
    const code = document.createElement('strong');
    code.textContent = diagnostic.code;
    const position = document.createElement('span');
    position.textContent = diagnostic.row && diagnostic.column
      ? 'Row ' + diagnostic.row + ', column ' + diagnostic.column
      : 'General';
    heading.append(code, position);
    const message = document.createElement('p');
    message.textContent = diagnostic.message;
    item.append(heading, message);
    elements.diagnostics.append(item);
  }
}

function renderPreview(inspection) {
  elements.previewHead.replaceChildren();
  elements.previewBody.replaceChildren();

  const columnLabels = inspection.headers.length > 0
    ? inspection.headers
    : Array.from({ length: inspection.column_count }, (_, index) => 'Column ' + (index + 1));
  const headRow = document.createElement('tr');
  headRow.append(makeCell('th', '#', 'col'));
  for (const label of columnLabels) headRow.append(makeCell('th', label, 'col'));
  elements.previewHead.append(headRow);

  for (const [index, row] of inspection.preview_rows.entries()) {
    const tableRow = document.createElement('tr');
    tableRow.append(makeCell('th', String(index + 1), 'row'));
    for (const value of row) tableRow.append(makeCell('td', value));
    elements.previewBody.append(tableRow);
  }
}

function renderInspection(inspection) {
  elements.rowCount.textContent = String(inspection.row_count);
  elements.columnCount.textContent = String(inspection.column_count);
  elements.selectedDelimiter.textContent = inspection.delimiter === '\t' ? 'Tab' : inspection.delimiter;
  elements.diagnosticCount.textContent = String(inspection.diagnostics.length);
  renderDiagnostics(inspection.diagnostics);
  renderPreview(inspection);
  elements.emptyState.hidden = true;
  elements.resultContent.hidden = false;
  elements.resultState.textContent = inspection.diagnostics.length > 0 ? 'Review findings' : 'No findings';
  elements.resultState.dataset.tone = inspection.diagnostics.length > 0 ? 'warning' : 'success';
}

function runInspection() {
  clearError();
  const response = inspect_csv(elements.input.value, inspectOptions());
  if (!response?.ok) {
    hasInspection = false;
    elements.resultContent.hidden = true;
    elements.emptyState.hidden = false;
    setError(response?.error ?? { code: 'WASM_RESPONSE_ERROR', message: 'The inspector returned an unreadable response.' });
    updateActions();
    return;
  }

  hasInspection = true;
  renderInspection(response.data);
  elements.exportOutput.value = '';
  updateActions();
}

function runExport() {
  clearError();
  const response = convert_csv(elements.input.value, convertOptions());
  if (!response?.ok) {
    elements.exportOutput.value = '';
    setError(response?.error ?? { code: 'WASM_RESPONSE_ERROR', message: 'The exporter returned an unreadable response.' });
    return;
  }
  elements.exportOutput.value = response.data.content;
}

function resetInspection() {
  hasInspection = false;
  elements.input.value = '';
  elements.file.value = '';
  elements.exportOutput.value = '';
  elements.resultContent.hidden = true;
  elements.emptyState.hidden = false;
  elements.resultState.textContent = 'Waiting for input';
  delete elements.resultState.dataset.tone;
  clearError();
  updateActions();
}

elements.input.addEventListener('input', () => {
  invalidateInspection();
});
elements.file.addEventListener('change', async () => {
  const [file] = elements.file.files;
  if (!file) return;
  if (file.size > MAX_FILE_BYTES) {
    setError({ code: 'INPUT_TOO_LARGE', message: 'The selected file exceeds the 10 MiB limit.' });
    return;
  }
  try {
    const bytes = await file.arrayBuffer();
    elements.input.value = new TextDecoder('utf-8', { fatal: true }).decode(bytes);
    invalidateInspection();
  } catch {
    setError({ code: 'INVALID_UTF8', message: 'The selected file is not valid UTF-8 text.' });
  }
});
elements.clear.addEventListener('click', resetInspection);
elements.inspect.addEventListener('click', runInspection);
elements.export.addEventListener('click', runExport);
elements.delimiter.addEventListener('change', invalidateInspection);
elements.headers.addEventListener('change', invalidateInspection);
elements.outputFormat.addEventListener('change', () => {
  const jsonSelected = elements.outputFormat.value === 'json';
  if (jsonSelected) elements.formulaPolicy.value = 'preserve';
  elements.formulaPolicy.disabled = jsonSelected;
  elements.exportOutput.value = '';
});
elements.formulaPolicy.addEventListener('change', () => {
  elements.exportOutput.value = '';
});

try {
  await init();
  wasmReady = true;
  elements.status.textContent = 'Ready';
  elements.runtime.dataset.state = 'ready';
  updateActions();
} catch {
  elements.status.textContent = 'Unavailable';
  elements.runtime.dataset.state = 'error';
  setError({ code: 'WASM_INIT_FAILED', message: 'The local inspection engine could not be initialized.' });
}
