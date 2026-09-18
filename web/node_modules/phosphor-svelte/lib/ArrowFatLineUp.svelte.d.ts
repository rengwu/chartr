import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowFatLineUpIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowFatLineUp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowFatLineUp: Component<IconComponentProps, {}, "">;
type ArrowFatLineUp = ReturnType<typeof ArrowFatLineUp>;
export default ArrowFatLineUp;
