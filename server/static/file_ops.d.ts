export function createDirectory(name: string): Promise<boolean>;
export function createFile(name: string, content?: string): Promise<boolean>;
export function renameItem(oldName: string, newName: string): Promise<boolean>;
export function deleteItem(name: string): Promise<boolean>;
export function listDirectory(path?: string): Promise<Array<{ name: string; is_dir: boolean }>>;
