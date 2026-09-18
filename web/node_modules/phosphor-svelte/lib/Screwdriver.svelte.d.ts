import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ScrewdriverIcon` instead.
 *
 * @example
 * ```svelte
 * <Screwdriver color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Screwdriver: Component<IconComponentProps, {}, "">;
type Screwdriver = ReturnType<typeof Screwdriver>;
export default Screwdriver;
