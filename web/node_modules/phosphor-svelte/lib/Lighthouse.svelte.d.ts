import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LighthouseIcon` instead.
 *
 * @example
 * ```svelte
 * <Lighthouse color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Lighthouse: Component<IconComponentProps, {}, "">;
type Lighthouse = ReturnType<typeof Lighthouse>;
export default Lighthouse;
