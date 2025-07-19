// File operations for rouillecloud web UI
export async function createDirectory(name) {
    const res = await fetch('/api/v1/dir', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name }),
    });
    return res.ok;
}

export async function createFile(name, content = '') {
    const res = await fetch('/api/v1/file', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name, content }),
    });
    return res.ok;
}

export async function renameItem(oldName, newName) {
    const res = await fetch('/api/v1/rename', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ oldName, newName }),
    });
    return res.ok;
}

export async function deleteItem(name) {
    const res = await fetch('/api/v1/delete', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name }),
    });
    return res.ok;
}

export async function listDirectory(path = '/') {
    const res = await fetch(`/api/v1/list?path=${encodeURIComponent(path)}`);
    if (res.ok) return await res.json();
    return [];
}
