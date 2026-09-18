import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `EarIcon` instead.
 *
 * @example
 * ```svelte
 * <Ear color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Ear: Component<IconComponentProps, {}, "">;
type Ear = ReturnType<typeof Ear>;
export default Ear;
