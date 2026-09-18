import type { Component } from "svelte";
import type { IconContextProps } from "./shared.d.ts";

/**
 *
 * @example
 * ```svelte
 * <IconContext
 *   values={{
 *     color: "white",
 *     weight: "fill",
 *     size: "20px",
 *     mirrored: false
 *   }}
 * >
 *   <Acorn />
 * </IconContext>
 * ```
 */
declare const IconContext: Component<IconContextProps, {}, "">;
type IconContext = ReturnType<typeof IconContext>;
export default IconContext;
