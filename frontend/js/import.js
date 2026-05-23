// Importer view — accepts Webull CSVs (parser pending real sample).

export function renderImportView(mount) {
    mount.innerHTML = `
        <div class="dropzone" id="dz">
            Drop a Webull CSV here (Account Statement → Orders).<br>
            <small>Parser awaits real sample — uploads will return 501 until phase 2 lands.</small>
        </div>
        <input type="file" id="picker" class="hidden" accept=".csv">
    `;
    const dz = document.getElementById('dz');
    const picker = document.getElementById('picker');
    dz.addEventListener('click', () => picker.click());
    ['dragenter', 'dragover'].forEach(ev =>
        dz.addEventListener(ev, e => { e.preventDefault(); dz.classList.add('dragover'); }));
    ['dragleave', 'drop'].forEach(ev =>
        dz.addEventListener(ev, e => { e.preventDefault(); dz.classList.remove('dragover'); }));
    dz.addEventListener('drop', e => handle(e.dataTransfer.files));
    picker.addEventListener('change', () => handle(picker.files));
}

function handle(files) {
    if (!files || !files.length) return;
    // POST /api/imports — not yet wired in routes.rs.
    // Once webull parser lands, this becomes a multipart upload.
    alert(`Selected ${files[0].name} (${files[0].size} bytes). Upload endpoint pending.`);
}
