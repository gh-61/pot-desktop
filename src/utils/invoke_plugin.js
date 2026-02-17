import { appCacheDir, appConfigDir, join } from "@tauri-apps/api/path";
import { readFile, readTextFile } from "@tauri-apps/plugin-fs";
import { invoke } from "@tauri-apps/api/core";
import Database from "@tauri-apps/plugin-sql";
import CryptoJS from "crypto-js";
import { osType } from "./env";
import * as http from "@tauri-apps/plugin-http"

// v1 compatibility: plugins may use Body.json(), Body.text(), Body.bytes(), Body.form()
// In v2, fetch uses standard Request/Response API, so we shim Body for backward compatibility
const Body = {
    json: (data) => JSON.stringify(data),
    text: (data) => data,
    bytes: (data) => data,
    form: (data) => {
        if (data instanceof FormData) return data;
        const form = new FormData();
        for (const [key, value] of Object.entries(data)) {
            form.append(key, value);
        }
        return form;
    },
};

// v1 compatibility wrapper: plugins expect res.data instead of res.json()/res.text()
// and use Body.json() for request body, responseType option, etc.
async function v1CompatFetch(url, options = {}) {
    const fetchOptions = { ...options };

    // v1 plugins pass headers as an object, v2 fetch accepts that too - no change needed

    // Handle body - if it's already a string from Body.json(), set content-type
    if (typeof fetchOptions.body === 'string' && !fetchOptions.headers?.['Content-Type'] && !fetchOptions.headers?.['content-type']) {
        fetchOptions.headers = { ...fetchOptions.headers, 'Content-Type': 'application/json' };
    }

    // Remove v1-specific options that v2 fetch doesn't understand
    const responseType = fetchOptions.responseType;
    delete fetchOptions.responseType;
    delete fetchOptions.timeout;

    const res = await http.fetch(url, fetchOptions);

    // Build v1-compatible response with .data property
    let data;
    if (responseType === 3) {
        // Binary response
        const buffer = await res.arrayBuffer();
        data = Array.from(new Uint8Array(buffer));
    } else if (responseType === 2) {
        // Text response
        data = await res.text();
    } else {
        // Default: try JSON, fall back to text
        const text = await res.text();
        try {
            data = JSON.parse(text);
        } catch {
            data = text;
        }
    }

    return {
        ok: res.ok,
        status: res.status,
        headers: res.headers,
        data,
    };
}

const httpCompat = {
    ...http,
    fetch: v1CompatFetch,
    Body,
};

export async function invoke_plugin(pluginType, pluginName) {
    let configDir = await appConfigDir();
    let cacheDir = await appCacheDir();
    let pluginDir = await join(configDir, "plugins", pluginType, pluginName);
    let entryFile = await join(pluginDir, "main.js");
    let script = await readTextFile(entryFile);
    async function run(cmdName, args) {
        return await invoke("run_binary", {
            pluginType,
            pluginName,
            cmdName,
            args
        });
    }
    const utils = {
        tauriFetch: http.fetch,
        http: httpCompat,
        readFile,
        readTextFile,
        Database,
        CryptoJS,
        run,
        cacheDir, // String
        pluginDir, // String
        osType,// "Windows_NT", "Darwin", "Linux"
    }
    return [eval(`${script} ${pluginType}`), utils];
}