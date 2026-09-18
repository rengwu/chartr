import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PaperclipIcon` instead.
 *
 * @example
 * ```svelte
 * <Paperclip color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Paperclip: Component<IconComponentProps, {}, "">;
type Paperclip = ReturnType<typeof Paperclip>;
export default Paperclip;
