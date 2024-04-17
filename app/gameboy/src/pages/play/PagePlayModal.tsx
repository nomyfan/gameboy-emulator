import { PagePlay } from "./PagePlay";

export function PagePlayModal(props: { open?: boolean }) {
  if (!props.open) {
    return null;
  }

  return (
    <PagePlay
      style={{
        position: "fixed",
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
      }}
    />
  );
}
