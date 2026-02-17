import '@testing-library/jest-dom';

// Radix UI uses ResizeObserver internally (via @radix-ui/react-use-size)
global.ResizeObserver = class ResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
};
