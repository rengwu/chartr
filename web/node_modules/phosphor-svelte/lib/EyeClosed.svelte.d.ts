import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `EyeClosedIcon` instead.
 *
 * @example
 * ```svelte
 * <EyeClosed color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const EyeClosed: Component<IconComponentProps, {}, "">;
type EyeClosed = ReturnType<typeof EyeClosed>;
export default EyeClosed;
