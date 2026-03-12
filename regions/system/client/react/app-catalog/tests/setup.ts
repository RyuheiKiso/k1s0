import '@testing-library/jest-dom';
import { vi } from 'vitest';

vi.stubGlobal('open', vi.fn());
