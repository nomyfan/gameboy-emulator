import { useImmutableRef } from "@callcc/toolkit-js/react/useImmutableRef";
import { Button } from "gameboy/components/ui/button";
import { Slider } from "gameboy/components/ui/slider";
import { Switch } from "gameboy/components/ui/switch";
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "gameboy/components/ui/tabs";
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

const speedOptions = [0.5, 1, 1.5, 2, 3];

const tabsTriggerClassName =
  "justify-start rounded-none border-0 border-l-[3px] border-transparent px-3 py-1.5 text-base font-normal text-foreground after:hidden data-[state=active]:border-primary data-[state=active]:font-medium";

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
    <div className="w-screen h-screen bg-background">
      <Tabs
        orientation="vertical"
        defaultValue={ETabs.Mics}
        className="h-full w-full gap-5"
      >
        <TabsList variant="line" className="gap-0 p-0">
          <TabsTrigger
            value={ETabs.Controller}
            className={tabsTriggerClassName}
          >
            控制器
          </TabsTrigger>
          <TabsTrigger value={ETabs.Mics} className={tabsTriggerClassName}>
            其他
          </TabsTrigger>
        </TabsList>

        <TabsContent
          value={ETabs.Controller}
          className="flex flex-col py-1.5 text-base"
        >
          (Coming soon...)
        </TabsContent>
        <TabsContent
          value={ETabs.Mics}
          className="flex flex-col py-1.5 text-base"
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

            <label className="font-semibold text-lg">游戏速度</label>
            <div className="mb-2 flex gap-1.5">
              {speedOptions.map((speed) => (
                <Button
                  key={speed}
                  type="button"
                  size="sm"
                  variant={settings.speed === speed ? "default" : "outline"}
                  onClick={() => {
                    store.setState({ speed });
                  }}
                >
                  {speed}x
                </Button>
              ))}
            </div>

            <label className="font-semibold text-lg" htmlFor={`${id}autoPause`}>
              自动暂停
              <span className="text-xs">（当离开当前页面时自动暂停游戏）</span>
            </label>
            <Switch
              id={`${id}autoPause`}
              className="mb-2 flex"
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
              className="mb-2 flex"
              checked={!settings.coerceBwColors}
              onCheckedChange={(checked) => {
                store.setState({ coerceBwColors: !checked });
              }}
            />

            <label
              className="font-semibold text-lg"
              htmlFor={`${id}hideActionButtons`}
            >
              隐藏操控按钮
            </label>
            <Switch
              id={`${id}hideActionButtons`}
              className="flex"
              checked={settings.hideActionButtons}
              onCheckedChange={(checked) => {
                store.setState({ hideActionButtons: checked });
              }}
            />
          </form>

          <div className="flex flex-row-reverse gap-2.5 m-2.5">
            <Button onClick={flush}>保存</Button>
            <Button
              variant="secondary"
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
