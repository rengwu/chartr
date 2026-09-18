import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GlobeIcon` instead.
 *
 * @example
 * ```svelte
 * <Globe color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Globe: Component<IconComponentProps, {}, "">;
type Globe = ReturnType<typeof Globe>;
export default Globe;
