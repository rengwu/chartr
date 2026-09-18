import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WarningIcon` instead.
 *
 * @example
 * ```svelte
 * <Warning color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Warning: Component<IconComponentProps, {}, "">;
type Warning = ReturnType<typeof Warning>;
export default Warning;
