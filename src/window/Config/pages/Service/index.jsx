import { readDir, BaseDirectory, readTextFile, exists } from '@tauri-apps/plugin-fs';
import { listen } from '@tauri-apps/api/event';
import { useTranslation } from 'react-i18next';
import { Tabs, Tab } from '@nextui-org/react';
import { appConfigDir, join } from '@tauri-apps/api/path';
import { convertFileSrc } from '@tauri-apps/api/core';
import { info, warn } from '@tauri-apps/plugin-log';
import React, { useEffect, useState } from 'react';
import Translate from './Translate';
import Recognize from './Recognize';
import Collection from './Collection';
import Tts from './Tts';
import { ServiceType } from '../../../../utils/service_instance';

let unlisten = null;

export default function Service() {
    const [pluginList, setPluginList] = useState(null);
    const { t } = useTranslation();

    const loadPluginList = async () => {
        const serviceTypeList = ['translate', 'tts', 'recognize', 'collection'];
        let temp = {};
        for (const serviceType of serviceTypeList) {
            temp[serviceType] = {};
            try {
                const dirExists = await exists(`plugins/${serviceType}`, { baseDir: BaseDirectory.AppConfig });
                await info(`[loadPluginList] ${serviceType}: dirExists=${dirExists}`);
                if (dirExists) {
                    const plugins = await readDir(`plugins/${serviceType}`, { baseDir: BaseDirectory.AppConfig });
                    await info(`[loadPluginList] ${serviceType}: readDir returned ${plugins.length} entries: ${JSON.stringify(plugins.map(p => ({name: p.name, isFile: p.isFile, isDirectory: p.isDirectory})))}`);
                    for (const plugin of plugins) {
                        if (plugin.isFile) continue;
                        try {
                            const infoStr = await readTextFile(`plugins/${serviceType}/${plugin.name}/info.json`, {
                                baseDir: BaseDirectory.AppConfig,
                            });
                            let pluginInfo = JSON.parse(infoStr);
                            if ('icon' in pluginInfo) {
                                const appConfigDirPath = await appConfigDir();
                                const iconPath = await join(
                                    appConfigDirPath,
                                    `/plugins/${serviceType}/${plugin.name}/${pluginInfo.icon}`
                                );
                                pluginInfo.icon = convertFileSrc(iconPath);
                            }
                            temp[serviceType][plugin.name] = pluginInfo;
                            await info(`[loadPluginList] Loaded plugin: ${serviceType}/${plugin.name}`);
                        } catch (e) {
                            await warn(`[loadPluginList] Failed to load plugin ${plugin.name}: ${e}`);
                        }
                    }
                }
            } catch (e) {
                await warn(`[loadPluginList] Failed to load plugins for ${serviceType}: ${e}`);
            }
        }
        await info(`[loadPluginList] Final result: ${JSON.stringify(Object.keys(temp).map(k => `${k}: [${Object.keys(temp[k]).join(', ')}]`))}`);
        setPluginList({ ...temp });
    };

    useEffect(() => {
        loadPluginList();
        if (unlisten) {
            unlisten.then((f) => {
                f();
            });
        }
        unlisten = listen('reload_plugin_list', loadPluginList);
        return () => {
            if (unlisten) {
                unlisten.then((f) => {
                    f();
                });
            }
        };
    }, []);
    return (
        pluginList !== null && (
            <Tabs className='flex justify-center max-h-[calc(100%-40px)] overflow-y-auto'>
                <Tab
                    key='translate'
                    title={t(`config.service.translate`)}
                >
                    <Translate pluginList={pluginList[ServiceType.TRANSLATE]} />
                </Tab>
                <Tab
                    key='recognize'
                    title={t(`config.service.recognize`)}
                >
                    <Recognize pluginList={pluginList[ServiceType.RECOGNIZE]} />
                </Tab>
                <Tab
                    key='tts'
                    title={t(`config.service.tts`)}
                >
                    <Tts pluginList={pluginList[ServiceType.TTS]} />
                </Tab>
                <Tab
                    key='collection'
                    title={t(`config.service.collection`)}
                >
                    <Collection pluginList={pluginList[ServiceType.COLLECTION]} />
                </Tab>
            </Tabs>
        )
    );
}
