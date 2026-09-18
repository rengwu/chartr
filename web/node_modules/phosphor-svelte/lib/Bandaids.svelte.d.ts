import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BandaidsIcon` instead.
 *
 * @example
 * ```svelte
 * <Bandaids color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Bandaids: Component<IconComponentProps, {}, "">;
type Bandaids = ReturnType<typeof Bandaids>;
export default Bandaids;
