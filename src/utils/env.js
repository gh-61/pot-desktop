import { type, arch as archFn, version } from '@tauri-apps/plugin-os';
import { getVersion } from '@tauri-apps/api/app';

export let osType = '';
export let arch = '';
export let osVersion = '';
export let appVersion = '';

// Map v2 os type values to v1 values used throughout the codebase
const osTypeMap = {
    macos: 'Darwin',
    windows: 'Windows_NT',
    linux: 'Linux',
};

export async function initEnv() {
    const rawType = await type();
    osType = osTypeMap[rawType] || rawType;
    arch = await archFn();
    osVersion = await version();
    appVersion = await getVersion();
}
