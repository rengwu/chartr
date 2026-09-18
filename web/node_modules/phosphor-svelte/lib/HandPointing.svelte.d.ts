import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HandPointingIcon` instead.
 *
 * @example
 * ```svelte
 * <HandPointing color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HandPointing: Component<IconComponentProps, {}, "">;
type HandPointing = ReturnType<typeof HandPointing>;
export default HandPointing;
