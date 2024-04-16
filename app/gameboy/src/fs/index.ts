export async function rootDir() {
  return await navigator.storage.getDirectory();
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
  parent: FileSystemDirectoryHandle,
  name: string,
) {
  const children = name.split("/").filter(Boolean);
  if (!children.length) {
    return parent;
  }

  // TODO: detect if the sub-path type
  for (const child of children) {
    parent = await parent.getDirectoryHandle(child, { create: true });
  }

  return parent;
}

export async function createFile(
  parent: FileSystemDirectoryHandle,
  name: string,
) {
  const children = name.split("/").filter(Boolean);
  if (!children.length) {
    throw new Error("Empty filename");
  }

  // TODO: detect if the sub-path type
  for (const child of children.slice(0, -1)) {
    parent = await parent.getDirectoryHandle(child, { create: true });
  }
  return await parent.getFileHandle(children[children.length - 1], {
    create: true,
  });
}
