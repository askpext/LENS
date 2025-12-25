const dropZone = document.getElementById('dropZone');
const uploadSection = document.getElementById('uploadSection');
const loading = document.getElementById('loading');
const results = document.getElementById('results');
const error = document.getElementById('error');
const errorMessage = document.getElementById('errorMessage');
const analyzeAnother = document.getElementById('analyzeAnother');
const exportJson = document.getElementById('exportJson');

let currentVideoData = null;

// Initial check
window.addEventListener('DOMContentLoaded', async () => {
    try {
        await window.__TAURI_INTERNALS__.invoke('check_ffmpeg_status');
    } catch (err) {
        showError('FFMPEG_INIT_FAIL: ' + err);
    }
});

// Drop Handling
window.addEventListener('tauri://drag-drop', async (event) => {
    if (event.payload?.paths?.length > 0) analyzeVideo(event.payload.paths[0]);
});

dropZone.addEventListener('click', async () => {
    const path = await window.__TAURI_INTERNALS__.invoke('open_file_dialog');
    if (path) analyzeVideo(path);
});

dropZone.addEventListener('dragover', (e) => {
    e.preventDefault();
    dropZone.style.borderColor = '#fff';
});
dropZone.addEventListener('dragleave', () => {
    dropZone.style.borderColor = '#444';
});

async function analyzeVideo(path) {
    hideAll();
    loading.classList.remove('hidden');

    try {
        const data = await window.__TAURI_INTERNALS__.invoke('analyze_video', { path });
        currentVideoData = data;
        // Small artificial delay for effect (feels like processing)
        setTimeout(() => displayResults(data), 600);
    } catch (err) {
        showError('ANALYSIS_CORRUPT: ' + err);
    }
}

function displayResults(data) {
    hideAll();
    results.classList.remove('hidden');

    const video = data.video || {};
    const format = data.format || {};

    // --- 1. VISUALIZER ENGINE ---
    const width = video.width || 1920;
    const height = video.height || 1080;
    const aspectBox = document.getElementById('aspectBox');
    
    // Set aspect ratio using CSS
    aspectBox.style.aspectRatio = `${width} / ${height}`;
    
    // Logic to ensure box doesn't overflow container while keeping ratio
    if (width > height) {
        aspectBox.style.width = '100%';
        aspectBox.style.height = 'auto';
    } else {
        aspectBox.style.height = '100%';
        aspectBox.style.width = 'auto';
    }

    document.getElementById('resText').textContent = `${width}×${height}`;
    document.getElementById('aspectRatioText').textContent = video.aspect_ratio || "N/A";

    // --- 2. TECH BADGE ---
    const codec = video.codec || "UNK";
    document.getElementById('codecBadge').textContent = codec.toUpperCase().replace('H264', 'AVC').replace('H265', 'HEVC');
    document.getElementById('profileText').textContent = video.profile || "Main";
    document.getElementById('containerText').textContent = format.format_name ? format.format_name.toUpperCase() : "FILE";

    // --- 3. GAUGES ---
    // Calculate Bitrate Percentage (Ref Max: 50mbps for high end consumer video)
    const bitrate = parseInt(format.bit_rate) || 0;
    const maxRefBitrate = 50000000; // 50 mbps
    const percentage = Math.min((bitrate / maxRefBitrate) * 100, 100);
    
    document.getElementById('bitrateValue').textContent = formatBitrate(bitrate);
    // Use timeout to trigger CSS animation
    setTimeout(() => {
        document.getElementById('bitrateBar').style.width = `${percentage}%`;
        // Color logic: Red if low, White if good
        if(percentage < 10) document.getElementById('bitrateBar').style.backgroundColor = '#888'; 
        else document.getElementById('bitrateBar').style.backgroundColor = '#fff';
    }, 100);

    document.getElementById('sizeValue').textContent = formatBytes(format.size);

    // --- 4. STATS ---
    document.getElementById('durationText').textContent = formatDuration(format.duration);
    document.getElementById('fpsText').textContent = video.fps || "0";

    // --- 5. DETAILS LIST ---
    const list = document.getElementById('advancedInfo');
    list.innerHTML = '';
    addTechRow(list, 'PIXEL_FMT', video.pix_fmt);
    addTechRow(list, 'COLOR_SPACE', video.color_space);
    addTechRow(list, 'SCAN_TYPE', video.field_order || 'Progressive');
    addTechRow(list, 'REF_FRAMES', video.refs);
    addTechRow(list, 'BIT_DEPTH', video.bits_per_raw_sample ? `${video.bits_per_raw_sample}-bit` : '8-bit');

    // --- 6. AUDIO GRID ---
    const audioGrid = document.getElementById('audioGrid');
    audioGrid.innerHTML = '';
    if (data.audio && data.audio.length > 0) {
        data.audio.forEach((track, i) => {
            const div = document.createElement('div');
            div.className = 'audio-module';
            div.innerHTML = `
                <span class="audio-type">${(track.codec || 'UNK').toUpperCase()}</span>
                <span class="audio-meta">
                    ${track.channels}ch / ${track.sample_rate}Hz / ${track.language || 'und'}
                </span>
            `;
            audioGrid.appendChild(div);
        });
    } else {
        audioGrid.innerHTML = '<div class="audio-module" style="opacity:0.5">NO_AUDIO_STREAM</div>';
    }
}

// Helpers
function addTechRow(parent, label, value) {
    if(!value || value === 'unknown') return;
    const row = document.createElement('div');
    row.className = 'tech-row';
    row.innerHTML = `<span>${label}</span><span>${value}</span>`;
    parent.appendChild(row);
}

function formatDuration(seconds) {
    if (!seconds) return '--:--:--';
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = Math.floor(seconds % 60);
    return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}

function formatBytes(bytes) {
    if (!bytes) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB'];
    let size = bytes;
    let i = 0;
    while (size >= 1024 && i < units.length - 1) { size /= 1024; i++; }
    return `${size.toFixed(2)} ${units[i]}`;
}

function formatBitrate(bps) {
    if (!bps) return '0 kb/s';
    return `${(bps / 1000).toFixed(0)} kb/s`;
}

function showError(msg) {
    hideAll();
    error.classList.remove('hidden');
    errorMessage.textContent = msg;
}

function hideAll() {
    uploadSection.classList.add('hidden');
    loading.classList.add('hidden');
    results.classList.add('hidden');
    error.classList.add('hidden');
}

analyzeAnother.addEventListener('click', () => { hideAll(); uploadSection.classList.remove('hidden'); });
errorRetry.addEventListener('click', () => { hideAll(); uploadSection.classList.remove('hidden'); });

exportJson.addEventListener('click', async () => {
    if (!currentVideoData) return;
    try {
        await window.__TAURI_INTERNALS__.invoke('save_json', { data: JSON.stringify(currentVideoData, null, 2) });
    } catch (e) { alert('ERR_WRITE: ' + e); }
});