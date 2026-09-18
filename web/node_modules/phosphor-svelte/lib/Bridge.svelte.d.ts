import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BridgeIcon` instead.
 *
 * @example
 * ```svelte
 * <Bridge color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Bridge: Component<IconComponentProps, {}, "">;
type Bridge = ReturnType<typeof Bridge>;
export default Bridge;
