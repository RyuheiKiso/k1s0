import type { Meta, StoryObj } from '@storybook/react';
import { fn, expect, within, userEvent } from '@storybook/test';
import { z } from 'zod';
import { createFormFromSchema } from './createFormFromSchema';

const meta: Meta = {
  title: 'Components/FormGenerator',
  parameters: {
    layout: 'padded',
    docs: {
      description: {
        component: 'Zod スキーマから MUI フォームを自動生成するコンポーネント',
      },
    },
  },
  tags: ['autodocs'],
};

export default meta;

// =============================================================================
// 基本フォーム
// =============================================================================

const basicSchema = z.object({
  name: z.string().min(1, '名前は必須です'),
  email: z.string().email('有効なメールアドレスを入力してください'),
});

const BasicForm = createFormFromSchema(basicSchema, {
  labels: {
    name: '氏名',
    email: 'メールアドレス',
  },
  submitLabel: '送信',
});

export const Basic: StoryObj = {
  render: () => (
    <BasicForm
      defaultValues={{ name: '', email: '' }}
      onSubmit={fn()}
    />
  ),
  parameters: {
    docs: {
      description: {
        story: '最もシンプルなフォーム。テキストフィールドとメールフィールドのみ。',
      },
    },
  },
};

// =============================================================================
// 全 MUI フィールドタイプ
// =============================================================================

const allFieldsSchema = z.object({
  // テキスト系
  text: z.string().min(1, '必須項目です'),
  email: z.string().email('有効なメールアドレスを入力してください'),
  password: z.string().min(8, '8文字以上で入力してください'),
  multiline: z.string().max(500, '500文字以内で入力してください'),

  // 数値系
  number: z.number().min(0).max(100),
  age: z.number().int().min(0).max(120),

  // 選択系
  select: z.enum(['option1', 'option2', 'option3']),
  radio: z.enum(['male', 'female', 'other']),

  // ブール系
  checkbox: z.boolean(),
  switch: z.boolean().default(true),

  // 日付系
  date: z.date(),

  // スライダー・評価
  slider: z.number().min(0).max(100),
  rating: z.number().min(0).max(5),
});

const AllFieldsForm = createFormFromSchema(allFieldsSchema, {
  labels: {
    text: 'テキスト',
    email: 'メールアドレス',
    password: 'パスワード',
    multiline: '自己紹介',
    number: '数値',
    age: '年齢',
    select: 'セレクト',
    radio: '性別',
    checkbox: '利用規約に同意',
    switch: '通知を受け取る',
    date: '生年月日',
    slider: '満足度',
    rating: '評価',
  },
  fieldConfig: {
    password: { type: 'password' },
    multiline: { multiline: true, rows: 4 },
    select: {
      component: 'Select',
      options: [
        { label: 'オプション1', value: 'option1' },
        { label: 'オプション2', value: 'option2' },
        { label: 'オプション3', value: 'option3' },
      ],
    },
    radio: {
      component: 'RadioGroup',
      options: [
        { label: '男性', value: 'male' },
        { label: '女性', value: 'female' },
        { label: 'その他', value: 'other' },
      ],
    },
    checkbox: { component: 'Checkbox' },
    switch: { component: 'Switch' },
    date: { component: 'DatePicker' },
    slider: { component: 'Slider', min: 0, max: 100 },
    rating: { component: 'Rating' },
  },
  columns: 2,
  submitLabel: '保存',
  showCancel: true,
});

export const AllFieldTypes: StoryObj = {
  render: () => (
    <AllFieldsForm
      defaultValues={{
        text: '',
        email: '',
        password: '',
        multiline: '',
        number: 0,
        age: 25,
        select: 'option1',
        radio: 'male',
        checkbox: false,
        switch: true,
        date: new Date(),
        slider: 50,
        rating: 3,
      }}
      onSubmit={fn()}
      onCancel={fn()}
    />
  ),
  parameters: {
    docs: {
      description: {
        story: '全ての MUI フィールドタイプを含むフォーム',
      },
    },
  },
};

// =============================================================================
// バリデーション
// =============================================================================

const validationSchema = z.object({
  username: z
    .string()
    .min(3, 'ユーザー名は3文字以上必要です')
    .max(20, 'ユーザー名は20文字以内にしてください')
    .regex(/^[a-zA-Z0-9_]+$/, '英数字とアンダースコアのみ使用できます'),
  email: z.string().email('有効なメールアドレスを入力してください'),
  password: z
    .string()
    .min(8, 'パスワードは8文字以上必要です')
    .regex(/[A-Z]/, '大文字を1文字以上含めてください')
    .regex(/[a-z]/, '小文字を1文字以上含めてください')
    .regex(/[0-9]/, '数字を1文字以上含めてください'),
  age: z.number().min(18, '18歳以上である必要があります').max(120, '有効な年齢を入力してください'),
  website: z.string().url('有効なURLを入力してください').optional().or(z.literal('')),
});

const ValidationForm = createFormFromSchema(validationSchema, {
  labels: {
    username: 'ユーザー名',
    email: 'メールアドレス',
    password: 'パスワード',
    age: '年齢',
    website: 'ウェブサイト（任意）',
  },
  fieldConfig: {
    password: { type: 'password' },
    age: { type: 'number' },
  },
  submitLabel: '登録',
});

export const WithValidation: StoryObj = {
  render: () => (
    <ValidationForm
      defaultValues={{
        username: '',
        email: '',
        password: '',
        age: 0,
        website: '',
      }}
      onSubmit={fn()}
    />
  ),
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);

    // 空のまま送信を試みる
    const submitButton = canvas.getByRole('button', { name: /登録/i });
    await userEvent.click(submitButton);

    // エラーメッセージが表示されることを確認
    await expect(canvas.findByText(/ユーザー名は3文字以上必要です/i)).resolves.toBeInTheDocument();
  },
  parameters: {
    docs: {
      description: {
        story: '複雑なバリデーションルールを持つフォーム',
      },
    },
  },
};

// =============================================================================
// 条件付きフィールド
// =============================================================================

const conditionalSchema = z.object({
  userType: z.enum(['individual', 'business']),
  name: z.string().min(1, '名前は必須です'),
  companyName: z.string().optional(),
  taxId: z.string().optional(),
});

const ConditionalForm = createFormFromSchema(conditionalSchema, {
  labels: {
    userType: 'ユーザー種別',
    name: '氏名',
    companyName: '会社名',
    taxId: '法人番号',
  },
  fieldConfig: {
    userType: {
      component: 'RadioGroup',
      options: [
        { label: '個人', value: 'individual' },
        { label: '法人', value: 'business' },
      ],
    },
    companyName: {
      conditionalRender: (values) => values.userType === 'business',
    },
    taxId: {
      conditionalRender: (values) => values.userType === 'business',
    },
  },
  submitLabel: '次へ',
});

export const ConditionalFields: StoryObj = {
  render: () => (
    <ConditionalForm
      defaultValues={{
        userType: 'individual',
        name: '',
        companyName: '',
        taxId: '',
      }}
      onSubmit={fn()}
    />
  ),
  parameters: {
    docs: {
      description: {
        story: '「法人」を選択すると追加フィールドが表示される条件付きフォーム',
      },
    },
  },
};

// =============================================================================
// 配列フィールド
// =============================================================================

const arraySchema = z.object({
  teamName: z.string().min(1, 'チーム名は必須です'),
  members: z.array(
    z.object({
      name: z.string().min(1, 'メンバー名は必須です'),
      role: z.string().min(1, '役割は必須です'),
    })
  ).min(1, 'メンバーを1人以上追加してください'),
});

const ArrayForm = createFormFromSchema(arraySchema, {
  labels: {
    teamName: 'チーム名',
    members: 'メンバー',
  },
  fieldConfig: {
    members: {
      component: 'ArrayField',
      itemLabels: {
        name: '名前',
        role: '役割',
      },
      addLabel: 'メンバーを追加',
    },
  },
  submitLabel: 'チームを作成',
});

export const ArrayFields: StoryObj = {
  render: () => (
    <ArrayForm
      defaultValues={{
        teamName: '',
        members: [{ name: '', role: '' }],
      }}
      onSubmit={fn()}
    />
  ),
  parameters: {
    docs: {
      description: {
        story: '動的に追加・削除できる配列フィールドを含むフォーム',
      },
    },
  },
};

// =============================================================================
// 複数列レイアウト
// =============================================================================

const addressSchema = z.object({
  postalCode: z.string().regex(/^\d{3}-?\d{4}$/, '有効な郵便番号を入力してください'),
  prefecture: z.string().min(1, '都道府県は必須です'),
  city: z.string().min(1, '市区町村は必須です'),
  address1: z.string().min(1, '番地は必須です'),
  address2: z.string().optional(),
  building: z.string().optional(),
});

const AddressForm = createFormFromSchema(addressSchema, {
  labels: {
    postalCode: '郵便番号',
    prefecture: '都道府県',
    city: '市区町村',
    address1: '番地',
    address2: '丁目・番・号',
    building: '建物名・部屋番号',
  },
  fieldConfig: {
    prefecture: {
      component: 'Select',
      options: [
        { label: '東京都', value: 'tokyo' },
        { label: '神奈川県', value: 'kanagawa' },
        { label: '大阪府', value: 'osaka' },
        { label: '京都府', value: 'kyoto' },
        { label: '愛知県', value: 'aichi' },
      ],
    },
  },
  columns: 2,
  submitLabel: '保存',
});

export const MultiColumnLayout: StoryObj = {
  render: () => (
    <AddressForm
      defaultValues={{
        postalCode: '',
        prefecture: '',
        city: '',
        address1: '',
        address2: '',
        building: '',
      }}
      onSubmit={fn()}
    />
  ),
  parameters: {
    docs: {
      description: {
        story: '2列レイアウトの住所入力フォーム',
      },
    },
  },
};

// =============================================================================
// 初期値付き（編集モード）
// =============================================================================

const profileSchema = z.object({
  name: z.string().min(1, '名前は必須です'),
  email: z.string().email('有効なメールアドレスを入力してください'),
  bio: z.string().max(500).optional(),
  notifications: z.boolean().default(true),
});

const ProfileForm = createFormFromSchema(profileSchema, {
  labels: {
    name: '氏名',
    email: 'メールアドレス',
    bio: '自己紹介',
    notifications: 'メール通知を受け取る',
  },
  fieldConfig: {
    bio: { multiline: true, rows: 4 },
    notifications: { component: 'Switch' },
  },
  submitLabel: '更新',
  showCancel: true,
});

export const EditMode: StoryObj = {
  render: () => (
    <ProfileForm
      defaultValues={{
        name: '山田太郎',
        email: 'yamada@example.com',
        bio: 'こんにちは！フロントエンドエンジニアをしています。',
        notifications: true,
      }}
      onSubmit={fn()}
      onCancel={fn()}
    />
  ),
  parameters: {
    docs: {
      description: {
        story: '既存データを編集する場合のフォーム（初期値あり）',
      },
    },
  },
};

// =============================================================================
// ローディング状態
// =============================================================================

export const Loading: StoryObj = {
  render: () => (
    <BasicForm
      defaultValues={{ name: '', email: '' }}
      onSubmit={async () => {
        await new Promise((resolve) => setTimeout(resolve, 2000));
      }}
      loading
    />
  ),
  parameters: {
    docs: {
      description: {
        story: '送信中のローディング状態',
      },
    },
  },
};

// =============================================================================
// 無効化状態
// =============================================================================

export const Disabled: StoryObj = {
  render: () => (
    <ProfileForm
      defaultValues={{
        name: '山田太郎',
        email: 'yamada@example.com',
        bio: 'プロフィール情報',
        notifications: true,
      }}
      onSubmit={fn()}
      disabled
    />
  ),
  parameters: {
    docs: {
      description: {
        story: 'フォーム全体が無効化された状態（読み取り専用）',
      },
    },
  },
};

// =============================================================================
// 日付・時刻フィールド
// =============================================================================

const dateTimeSchema = z.object({
  date: z.date(),
  dateTime: z.date(),
  time: z.date(),
});

const DateTimeForm = createFormFromSchema(dateTimeSchema, {
  labels: {
    date: '日付',
    dateTime: '日時',
    time: '時刻',
  },
  fieldConfig: {
    date: { component: 'DatePicker' },
    dateTime: { component: 'DateTimePicker' },
    time: { component: 'TimePicker' },
  },
  submitLabel: '設定',
});

export const DateTimeFields: StoryObj = {
  render: () => (
    <DateTimeForm
      defaultValues={{
        date: new Date(),
        dateTime: new Date(),
        time: new Date(),
      }}
      onSubmit={fn()}
    />
  ),
  parameters: {
    docs: {
      description: {
        story: '日付・日時・時刻ピッカーを含むフォーム',
      },
    },
  },
};

// =============================================================================
// Autocomplete フィールド
// =============================================================================

const autocompleteSchema = z.object({
  country: z.string().min(1, '国を選択してください'),
  tags: z.array(z.string()).min(1, 'タグを1つ以上選択してください'),
});

const AutocompleteForm = createFormFromSchema(autocompleteSchema, {
  labels: {
    country: '国',
    tags: 'タグ',
  },
  fieldConfig: {
    country: {
      component: 'Autocomplete',
      options: [
        { label: '日本', value: 'jp' },
        { label: 'アメリカ', value: 'us' },
        { label: 'イギリス', value: 'uk' },
        { label: 'フランス', value: 'fr' },
        { label: 'ドイツ', value: 'de' },
        { label: '中国', value: 'cn' },
        { label: '韓国', value: 'kr' },
      ],
    },
    tags: {
      component: 'Autocomplete',
      multiple: true,
      options: [
        { label: 'React', value: 'react' },
        { label: 'TypeScript', value: 'typescript' },
        { label: 'Node.js', value: 'nodejs' },
        { label: 'Python', value: 'python' },
        { label: 'Go', value: 'go' },
        { label: 'Rust', value: 'rust' },
      ],
    },
  },
  submitLabel: '保存',
});

export const AutocompleteFields: StoryObj = {
  render: () => (
    <AutocompleteForm
      defaultValues={{
        country: '',
        tags: [],
      }}
      onSubmit={fn()}
    />
  ),
  parameters: {
    docs: {
      description: {
        story: 'オートコンプリート（単一選択・複数選択）を含むフォーム',
      },
    },
  },
};
