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
      resolve(file);
    });

    /**
     * @see https://caniuse.com/mdn-api_htmlinputelement_cancel_event
     */
    input.addEventListener("cancel", () => {
      reject();
    });

    input.click();
    input.remove();
  });
}
