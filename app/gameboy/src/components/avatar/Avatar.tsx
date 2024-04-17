import * as styles from "./Avatar.css";
import MockAvatar from "./mock.jpg";

export function Avatar() {
  return (
    <div className={styles.avatar}>
      <img
        alt="avatar"
        src={MockAvatar}
        style={{
          height: "100%",
          width: "100%",
        }}
      />
    </div>
  );
}
