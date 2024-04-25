/**
 * @param options
 */
export async function pickFile(options?: {
  accept?: string;
  multiple?: boolean;
}) {
  const input = document.createElement("input");
  input.accept = options?.accept ?? "*/*";
  input.type = "file";
  if (options?.multiple) {
    input.setAttribute("multiple", "");
  }
  input.style.display = "none";
  document.body.appendChild(input);

  return new Promise<FileList | null>((resolve, reject) => {
    input.addEventListener("change", () => {
      resolve(input.files);
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
