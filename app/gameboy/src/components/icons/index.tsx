import { default as IconChecked } from "./checked.svg?react";
import { default as IconIndeterminate } from "./indeterminate.svg?react";
import { default as IconUnchecked } from "./unchecked.svg?react";

export { default as IconDelete } from "./delete_forever.svg?react";
export { default as IconPlay } from "./play_arrow.svg?react";
export { default as IconHistory } from "./manage_history.svg?react";
export { default as IconAdd } from "./playlist_add.svg?react";
export { default as IconSettings } from "./settings.svg?react";
export { default as IconFullscreen } from "./fullscreen.svg?react";
export { default as IconFullscreenExit } from "./fullscreen_exit.svg?react";
export { default as IconVolumeOff } from "./volume_off.svg?react";
export { default as IconVolumeOn } from "./volume_up.svg?react";
export { default as IconFileDownload } from "./file_download.svg?react";
export { default as IconFileUpload } from "./file_upload.svg?react";
export { default as IconExpandDown } from "./expand_more.svg?react";
export { default as IconSave } from "./save.svg?react";
export { default as IconPause } from "./pause.svg?react";
export { default as IconExitToApp } from "./exit_to_app.svg?react";
export { IconChecked, IconUnchecked, IconIndeterminate };

export function IconCheck(
  props: Parameters<typeof IconChecked>[0] & {
    checked: boolean | "indeterminate";
  },
) {
  const { checked, ...restProps } = props;
  if (checked === "indeterminate") {
    return <IconIndeterminate {...restProps} />;
  } else if (checked) {
    return <IconChecked {...restProps} />;
  } else {
    return <IconUnchecked {...restProps} />;
  }
}
