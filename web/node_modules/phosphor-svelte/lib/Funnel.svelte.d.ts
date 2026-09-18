import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FunnelIcon` instead.
 *
 * @example
 * ```svelte
 * <Funnel color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Funnel: Component<IconComponentProps, {}, "">;
type Funnel = ReturnType<typeof Funnel>;
export default Funnel;
