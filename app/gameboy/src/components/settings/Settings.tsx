import { Tabs, TabsList, TabsTrigger, TabsContent } from "@radix-ui/react-tabs";
import { Slider } from "gameboy/components/core/slider";
import { store, actions } from "gameboy/store";
import { create } from "gameboy/store/utils";
import { cloneDeep } from "gameboy/utils";
import { cn } from "gameboy/utils/cn";
import { useState } from "react";
import { useStore } from "zustand";

import { Button } from "../core/button";

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
    <div className="w-screen h-screen bg-bg/75 backdrop-blur-lg">
      <Tabs
        orientation="vertical"
        defaultValue={ETabs.Mics}
        className="h-full w-full flex gap-5"
      >
        <TabsList>
          <TabsTrigger
            value={ETabs.Controller}
            className={cn(
              "block w-full text-left border-l-[3px] border-solid border-transparent px-3 py-1.5",
              "data-[state=active]:border-primary",
            )}
          >
            控制器
          </TabsTrigger>
          <TabsTrigger
            value={ETabs.Mics}
            className={cn(
              "block w-full text-left border-l-[3px] border-solid border-transparent px-3 py-1",
              "data-[state=active]:border-primary",
            )}
          >
            其他
          </TabsTrigger>
        </TabsList>

        <TabsContent
          value={ETabs.Controller}
          className={cn(
            "flex flex-col grow py-1.5 outline-0",
            "data-[state=inactive]:hidden",
          )}
        >
          TBD
        </TabsContent>
        <TabsContent
          value={ETabs.Mics}
          className={cn(
            "flex flex-col grow py-1.5 outline-0",
            "data-[state=inactive]:hidden",
          )}
        >
          <div style={{ flexGrow: 1 }}>
            <label className="font-semibold text-lg mb-2.5">
              游戏音量（{settings.volume}%）
            </label>
            <Slider
              style={{ width: "200px" }}
              value={[settings.volume]}
              onValueChange={([volume]) => {
                settingsStore.setState((state) => {
                  state.volume = volume;
                });
              }}
            />
          </div>

          <div className="flex flex-row-reverse gap-2.5 m-2.5">
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
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}
