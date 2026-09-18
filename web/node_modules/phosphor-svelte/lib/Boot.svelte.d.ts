import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BootIcon` instead.
 *
 * @example
 * ```svelte
 * <Boot color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Boot: Component<IconComponentProps, {}, "">;
type Boot = ReturnType<typeof Boot>;
export default Boot;
