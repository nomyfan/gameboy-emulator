import * as styles from "./Snapshots.css";

export function Snapshots() {
  const renderItems = () => {
    return Array.from({ length: 30 }, (_, i) => {
      return (
        <div key={i} style={{ height: "20px" }}>
          {i}
        </div>
      );
    });
  };

  return (
    <div className={styles.snapshotsRoot}>
      <h1 className={styles.header}>存档</h1>

      <div className={styles.itemsContainer}>{renderItems()}</div>
    </div>
  );
}
