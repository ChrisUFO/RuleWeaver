import React from "react";
import { render } from "@testing-library/react";
import { ToastProvider } from "../../components/ui/toast";

/**
 * Wrap a component in the providers all lifecycle tests need (currently just ToastProvider).
 * Import this helper in every lifecycle test file instead of duplicating the wrapper.
 */
export const renderWithProviders = (ui: React.ReactElement) =>
  render(<ToastProvider>{ui}</ToastProvider>);
