import { useImmutableRef } from "@callcc/toolkit-js/react/useImmutableRef";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@radix-ui/react-tabs";
import { clsx } from "clsx";
import { Button } from "gameboy/components/core/button";
import { Slider } from "gameboy/components/core/slider";
import { Switch } from "gameboy/components/core/switch";
import { closeSettingsModal } from "gameboy/store/app";
import { settingsStore, writeSettings } from "gameboy/store/settings";
import { create } from "gameboy/store/utils";
import { cloneDeep } from "gameboy/utils";
import { useId } from "react";
import { useStore } from "zustand";

enum ETabs {
  Controller = "controller",
  Mics = "misc",
}

export function Settings() {
  const store = useImmutableRef(() =>
    create(() => cloneDeep(settingsStore.getState())),
  );
  const settings = useStore(store);
  const id = useId();

  const flush = () => {
    writeSettings(cloneDeep(settings));
    closeSettingsModal();
  };

  return (
    <div className="w-screen h-screen bg-bg">
      <Tabs
        orientation="vertical"
        defaultValue={ETabs.Mics}
        className="h-full w-full flex gap-5"
      >
        <TabsList>
          <TabsTrigger
            value={ETabs.Controller}
            className={clsx(
              "block w-full text-left border-l-[3px] border-solid border-transparent px-3 py-1.5",
              "[&[data-state=active]]:(border-primary font-medium)",
            )}
          >
            控制器
          </TabsTrigger>
          <TabsTrigger
            value={ETabs.Mics}
            className={clsx(
              "block w-full text-left border-l-[3px] border-solid border-transparent px-3 py-1.5",
              "[&[data-state=active]]:(border-primary font-medium)",
            )}
          >
            其他
          </TabsTrigger>
        </TabsList>

        <TabsContent
          value={ETabs.Controller}
          className={clsx(
            "flex flex-col grow py-1.5 outline-0",
            "[&[data-state=inactive]]:hidden",
          )}
        >
          (Coming soon...)
        </TabsContent>
        <TabsContent
          value={ETabs.Mics}
          className={clsx(
            "flex flex-col grow py-1.5 outline-0",
            "[&[data-state=inactive]]:hidden",
          )}
        >
          <form style={{ flexGrow: 1 }}>
            <label className="font-semibold text-lg" htmlFor={`${id}-volume`}>
              游戏音量（{settings.volume}%）
            </label>
            <Slider
              className="mb-2"
              style={{ width: "200px" }}
              value={[settings.volume]}
              onValueChange={([volume]) => {
                store.setState({ volume });
              }}
              id={`${id}-volume`}
            />

            <label className="font-semibold text-lg" htmlFor={`${id}autoPause`}>
              自动暂停
              <span className="text-xs">（当离开当前页面时自动暂停游戏）</span>
            </label>
            <Switch
              id={`${id}autoPause`}
              className="mb-2"
              checked={settings.autoPause}
              onCheckedChange={(checked) => {
                store.setState({ autoPause: checked });
              }}
            />

            <label className="font-semibold text-lg" htmlFor={`${id}coerceBW`}>
              开启 DMG 色彩兼容模式
            </label>
            <Switch
              id={`${id}coerceBW`}
              checked={!settings.coerceBwColors}
              onCheckedChange={(checked) => {
                store.setState({ coerceBwColors: !checked });
              }}
            />
          </form>

          <div className="flex flex-row-reverse gap-2.5 m-2.5">
            <Button variant="primary" onClick={flush}>
              保存
            </Button>
            <Button
              onClick={() => {
                closeSettingsModal();
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
