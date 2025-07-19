// SPA navigation
const sections = {
    files: document.getElementById('file-browser'),
    users: document.getElementById('user-panel'),
    admin: document.getElementById('admin-panel'),
};
function showSection(name) {
    Object.values(sections).forEach(s => s.classList.add('hidden'));
    sections[name].classList.remove('hidden');
    sections[name].classList.add('active');
}
document.getElementById('btn-files').onclick = () => showSection('files');
document.getElementById('btn-users').onclick = () => showSection('users');
document.getElementById('btn-admin').onclick = () => showSection('admin');

// Theme toggle
const app = document.getElementById('app');
const themeBtn = document.getElementById('btn-theme');
let dark = false;
themeBtn.onclick = () => {
    dark = !dark;
    app.setAttribute('data-theme', dark ? 'dark' : '');
};

// Drag & drop upload
const dropZone = document.getElementById('drop-zone');
dropZone.addEventListener('dragover', e => {
    e.preventDefault();
    dropZone.style.background = '#e3f2fd';
});
dropZone.addEventListener('dragleave', e => {
    dropZone.style.background = '#fff';
});
dropZone.addEventListener('drop', async e => {
    e.preventDefault();
    dropZone.style.background = '#fff';
    const files = Array.from(e.dataTransfer.files);
    for (const file of files) {
        await uploadFile(file);
    }
});

async function uploadFile(file) {
    const formData = new FormData();
    formData.append('file', file);
    const res = await fetch('/api/v1/upload', {
        method: 'POST',
        body: formData,
    });
    if (res.ok) {
        refreshFileList();
    } else {
        alert('Upload failed');
    }
}
function addFileToList(name) {
    const li = document.createElement('li');
    li.textContent = name;
    document.getElementById('file-list').appendChild(li);
}

// WebSocket sync status
const syncStatus = document.getElementById('sync-status');
let ws;
function connectWS() {
    ws = new WebSocket('ws://' + location.host + '/api/v1/ws');
    ws.onopen = () => syncStatus.textContent = 'Sync: Connected';
    ws.onclose = () => syncStatus.textContent = 'Sync: Disconnected';
    ws.onmessage = e => {
        // TODO: Handle real-time updates
        console.log('WS:', e.data);
    };
}
connectWS();

import { createDirectory, createFile, renameItem, deleteItem, listDirectory } from './file_ops.js';

let currentPath = '/';
const currentPathSpan = document.getElementById('current-path');
const fileList = document.getElementById('file-list');

async function refreshFileList() {
    const items = await listDirectory(currentPath);
    fileList.innerHTML = '';
    for (const item of items) {
        const li = document.createElement('li');
        li.textContent = item.name + (item.is_dir ? '/': '');
        li.onclick = () => {
            if (item.is_dir) {
                currentPath = currentPath.endsWith('/') ? currentPath + item.name : currentPath + '/' + item.name;
                currentPathSpan.textContent = currentPath;
                refreshFileList();
            }
        };
        fileList.appendChild(li);
    }
}

document.getElementById('btn-create-dir').onclick = async () => {
    const name = document.getElementById('new-dir-name').value;
    if (name) {
        await createDirectory(currentPath.endsWith('/') ? currentPath + name : currentPath + '/' + name);
        refreshFileList();
    }
};
document.getElementById('btn-create-file').onclick = async () => {
    const name = document.getElementById('new-file-name').value;
    if (name) {
        await createFile(currentPath.endsWith('/') ? currentPath + name : currentPath + '/' + name);
        refreshFileList();
    }
};
document.getElementById('btn-rename').onclick = async () => {
    const oldName = document.getElementById('rename-old').value;
    const newName = document.getElementById('rename-new').value;
    if (oldName && newName) {
        await renameItem(currentPath.endsWith('/') ? currentPath + oldName : currentPath + '/' + oldName, currentPath.endsWith('/') ? currentPath + newName : currentPath + '/' + newName);
        refreshFileList();
    }
};
document.getElementById('btn-delete').onclick = async () => {
    const name = document.getElementById('delete-name').value;
    if (name) {
        await deleteItem(currentPath.endsWith('/') ? currentPath + name : currentPath + '/' + name);
        refreshFileList();
    }
};
document.getElementById('btn-up').onclick = () => {
    if (currentPath !== '/') {
        const parts = currentPath.split('/').filter(Boolean);
        parts.pop();
        currentPath = '/' + parts.join('/');
        if (!currentPath.endsWith('/')) currentPath += '/';
        currentPathSpan.textContent = currentPath;
        refreshFileList();
    }
};

window.addEventListener('DOMContentLoaded', refreshFileList);
