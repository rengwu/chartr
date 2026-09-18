import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CheersIcon` instead.
 *
 * @example
 * ```svelte
 * <Cheers color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Cheers: Component<IconComponentProps, {}, "">;
type Cheers = ReturnType<typeof Cheers>;
export default Cheers;
