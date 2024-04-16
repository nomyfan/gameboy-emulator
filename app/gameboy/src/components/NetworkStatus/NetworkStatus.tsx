import { useNetworkState } from "react-use";

import { IconWifi0, IconWifi1, IconWifi2, IconWifi3, IconWifi } from "../Icons";

export function NetworkStatus() {
  const { effectiveType, type, online } = useNetworkState();

  if (!online) {
    return <IconWifi0 />;
  }

  if (effectiveType === "slow-2g") {
    return <IconWifi1 />;
  }

  if (effectiveType === "2g") {
    return <IconWifi2 />;
  }

  if (effectiveType === "3g") {
    return <IconWifi3 />;
  }

  return <IconWifi />;
}
