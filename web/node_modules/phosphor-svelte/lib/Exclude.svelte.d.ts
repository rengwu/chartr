import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ExcludeIcon` instead.
 *
 * @example
 * ```svelte
 * <Exclude color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Exclude: Component<IconComponentProps, {}, "">;
type Exclude = ReturnType<typeof Exclude>;
export default Exclude;
