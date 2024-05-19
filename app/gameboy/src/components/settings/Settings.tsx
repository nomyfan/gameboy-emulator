import {
  Slider,
  SliderTrack,
  SliderThumb,
  SliderRange,
} from "@radix-ui/react-slider";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@radix-ui/react-tabs";
import { FlexBox } from "gameboy/components/core/flex-box";
import { store, actions } from "gameboy/store";
import { create } from "gameboy/store/utils";
import { cloneDeep } from "gameboy/utils";
import { useState } from "react";
import { useStore } from "zustand";

import { Button } from "../core/button";

import * as styles from "./Settings.css";

enum ETabs {
  Controller = "controller",
  Mics = "misc",
}

export function Settings() {
  const [settingsStore] = useState(() => {
    return create(() => cloneDeep(store.getState().settings));
  });
  const settings = useStore(settingsStore);

  const flush = () => {
    actions.writeSettings(cloneDeep(settingsStore.getState()));
    actions.closeSettingsModal();
  };

  return (
    <div className={styles.container}>
      <Tabs
        orientation="vertical"
        defaultValue={ETabs.Mics}
        className={styles.tabs}
      >
        <TabsList className={styles.list}>
          <TabsTrigger value={ETabs.Controller} className={styles.trigger}>
            控制器
          </TabsTrigger>
          <TabsTrigger value={ETabs.Mics} className={styles.trigger}>
            其他
          </TabsTrigger>
        </TabsList>

        <TabsContent value={ETabs.Controller} className={styles.content}>
          TBD
        </TabsContent>
        <TabsContent value={ETabs.Mics} className={styles.content}>
          <div style={{ flexGrow: 1 }}>
            <label
              style={{
                marginBottom: 10,
                fontSize: "1.17em",
                fontWeight: "bolder",
              }}
            >
              游戏音量（{settings.volume}%）
            </label>
            <Slider
              className={styles.slider}
              value={[settings.volume]}
              onValueChange={([volume]) => {
                settingsStore.setState((state) => {
                  state.volume = volume;
                });
              }}
            >
              <SliderTrack className={styles.track}>
                <SliderRange className={styles.range} />
              </SliderTrack>

              <SliderThumb className={styles.thumb} />
            </Slider>
          </div>

          <FlexBox direction="row-reverse" gap={10} style={{ margin: 10 }}>
            <Button type="primary" onClick={flush}>
              保存
            </Button>
            <Button
              onClick={() => {
                actions.closeSettingsModal();
              }}
            >
              返回
            </Button>
          </FlexBox>
        </TabsContent>
      </Tabs>
    </div>
  );
}
