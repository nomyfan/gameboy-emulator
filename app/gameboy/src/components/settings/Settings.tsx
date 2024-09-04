import { cn } from "@callcc/toolkit-js/cn";
import { Switch, SwitchThumb } from "@radix-ui/react-switch";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@radix-ui/react-tabs";
import { Button } from "gameboy/components/core/button";
import { Slider } from "gameboy/components/core/slider";
import { store, actions } from "gameboy/store";
import { create } from "gameboy/store/utils";
import { cloneDeep } from "gameboy/utils";
import { useId, useState } from "react";
import { useStore } from "zustand";

enum ETabs {
  Controller = "controller",
  Mics = "misc",
}

export function Settings() {
  const [settingsStore] = useState(() => {
    return create(() => cloneDeep(store.getState().settings));
  });
  const settings = useStore(settingsStore);
  const id = useId();

  const flush = () => {
    actions.writeSettings(cloneDeep(settingsStore.getState()));
    actions.closeSettingsModal();
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
            className={cn(
              "block w-full text-left border-l-[3px] border-solid border-transparent px-3 py-1.5",
              "[&[data-state=active]]:(border-primary font-medium)",
            )}
          >
            控制器
          </TabsTrigger>
          <TabsTrigger
            value={ETabs.Mics}
            className={cn(
              "block w-full text-left border-l-[3px] border-solid border-transparent px-3 py-1.5",
              "[&[data-state=active]]:(border-primary font-medium)",
            )}
          >
            其他
          </TabsTrigger>
        </TabsList>

        <TabsContent
          value={ETabs.Controller}
          className={cn(
            "flex flex-col grow py-1.5 outline-0",
            "[&[data-state=inactive]]:hidden",
          )}
        >
          (Coming soon...)
        </TabsContent>
        <TabsContent
          value={ETabs.Mics}
          className={cn(
            "flex flex-col grow py-1.5 outline-0",
            "[&[data-state=inactive]]:hidden",
          )}
        >
          <form style={{ flexGrow: 1 }}>
            <label
              className="font-semibold text-lg mb-2.5"
              htmlFor={id + "-volume"}
            >
              游戏音量（{settings.volume}%）
            </label>
            <Slider
              style={{ width: "200px" }}
              value={[settings.volume]}
              onValueChange={([volume]) => {
                settingsStore.setState({ volume });
              }}
              id={id + "-volume"}
            />

            <label
              className="font-semibold text-lg mb-2.5"
              htmlFor={id + "autoPause"}
            >
              自动暂停
              <span className="text-xs">（当离开当前页面时自动暂停游戏）</span>
            </label>
            <Switch
              id={id + "autoPause"}
              className={cn(
                "w-11 h-6 rounded-full relative block bg-primary/70",
                "[&[data-state=checked]]:bg-primary",
              )}
              checked={settings.autoPause}
              onCheckedChange={(checked) => {
                settingsStore.setState({ autoPause: checked });
              }}
            >
              <SwitchThumb
                className={cn(
                  "block h-5 w-5 bg-white rounded-full transform-translate-x-0.5 transition-transform transition-duration-200",
                  "[&[data-state=checked]]:transform-translate-x-5.5",
                )}
              />
            </Switch>
          </form>

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
