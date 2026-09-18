import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CodepenLogoIcon` instead.
 *
 * @example
 * ```svelte
 * <CodepenLogo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CodepenLogo: Component<IconComponentProps, {}, "">;
type CodepenLogo = ReturnType<typeof CodepenLogo>;
export default CodepenLogo;
