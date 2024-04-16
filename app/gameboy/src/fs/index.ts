export async function rootDir() {
  return await navigator.storage.getDirectory();
}

export async function getParentDir(
  path: string,
  parent?: FileSystemDirectoryHandle,
) {
  if (path.startsWith("/")) {
    if (parent) {
      console.warn("`parent` will be ignored if `path` is absolute");
    }
    parent = await rootDir();
  } else {
    if (!parent) {
      throw new Error("Parent directory is required");
    }
  }

  return parent;
}

/**
 * @param options
 */
export async function pickFile(options?: { accept?: string }) {
  const input = document.createElement("input");
  input.accept = options?.accept ?? "*/*";
  input.type = "file";
  input.style.display = "none";
  document.body.appendChild(input);

  return new Promise<File | null>((resolve, reject) => {
    input.addEventListener("change", () => {
      const file = input.files?.[0] ?? null;
      input.remove();
      resolve(file);
    });

    /**
     * @see https://caniuse.com/mdn-api_htmlinputelement_cancel_event
     */
    input.addEventListener("cancel", () => {
      input.remove();
      reject();
    });

    input.click();
  });
}

export async function createDir(
  path: string,
  parent?: FileSystemDirectoryHandle,
) {
  parent = await getParentDir(path, parent);

  const children = path.split("/").filter(Boolean);
  if (!children.length) {
    return parent;
  }

  for (const child of children) {
    parent = await parent.getDirectoryHandle(child, { create: true });
  }

  return parent;
}

export async function createFile(
  path: string,
  parent?: FileSystemDirectoryHandle,
) {
  parent = await getParentDir(path, parent);

  const children = path.split("/").filter(Boolean);
  if (!children.length) {
    throw new Error("Empty path");
  }

  for (const child of children.slice(0, -1)) {
    parent = await parent.getDirectoryHandle(child, { create: true });
  }
  return await parent.getFileHandle(children[children.length - 1], {
    create: true,
  });
}

/**
 * @param path If your path is directory, please make sure it ends with `/`
 * @param parent
 */
export async function exists(path: string, parent?: FileSystemDirectoryHandle) {
  parent = await getParentDir(path, parent);

  const children = path.split("/").filter(Boolean);
  if (!children.length) {
    throw new Error("Empty path");
  }

  const isDir = path.endsWith("/");

  for (let i = 0; i < children.length - 1; i++) {
    try {
      parent = await parent.getDirectoryHandle(children[i]);
    } catch (err) {
      if (err instanceof DOMException && err.name === "NotFoundError") {
        return false;
      }
      throw err;
    }
  }

  try {
    if (isDir) {
      await parent.getDirectoryHandle(children[children.length - 1]);
    } else {
      await parent.getFileHandle(children[children.length - 1]);
    }
  } catch (err) {
    if (err instanceof DOMException && err.name === "NotFoundError") {
      return false;
    }
    throw err;
  }

  return true;
}
