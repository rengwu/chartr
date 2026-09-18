import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ClipboardTextIcon` instead.
 *
 * @example
 * ```svelte
 * <ClipboardText color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ClipboardText: Component<IconComponentProps, {}, "">;
type ClipboardText = ReturnType<typeof ClipboardText>;
export default ClipboardText;
