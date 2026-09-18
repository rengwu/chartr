import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BinocularsIcon` instead.
 *
 * @example
 * ```svelte
 * <Binoculars color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Binoculars: Component<IconComponentProps, {}, "">;
type Binoculars = ReturnType<typeof Binoculars>;
export default Binoculars;
