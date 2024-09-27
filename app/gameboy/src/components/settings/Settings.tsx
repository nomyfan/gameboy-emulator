import { useImmutableRef } from "@callcc/toolkit-js/react/useImmutableRef";
import { Switch, SwitchThumb } from "@radix-ui/react-switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@radix-ui/react-tabs";
import { clsx } from "clsx";
import { Button } from "gameboy/components/core/button";
import { Slider } from "gameboy/components/core/slider";
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
            <label
              className="font-semibold text-lg mb-2.5"
              htmlFor={`${id}-volume`}
            >
              游戏音量（{settings.volume}%）
            </label>
            <Slider
              style={{ width: "200px" }}
              value={[settings.volume]}
              onValueChange={([volume]) => {
                store.setState({ volume });
              }}
              id={`${id}-volume`}
            />

            <label
              className="font-semibold text-lg mb-2.5"
              htmlFor={`${id}autoPause`}
            >
              自动暂停
              <span className="text-xs">（当离开当前页面时自动暂停游戏）</span>
            </label>
            <Switch
              id={`${id}autoPause`}
              className={clsx(
                "w-11 h-6 rounded-full relative block bg-primary/70",
                "[&[data-state=checked]]:bg-primary",
              )}
              checked={settings.autoPause}
              onCheckedChange={(checked) => {
                store.setState({ autoPause: checked });
              }}
            >
              <SwitchThumb
                className={clsx(
                  "block h-5 w-5 bg-white rounded-full transform-translate-x-0.5 transition-transform transition-duration-200",
                  "[&[data-state=checked]]:transform-translate-x-5.5",
                )}
              />
            </Switch>
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
