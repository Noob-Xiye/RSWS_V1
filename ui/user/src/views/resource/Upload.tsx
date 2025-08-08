import React, { useState, useEffect } from 'react';
import { Form, Input, Select, InputNumber, Upload, Button, Card, Steps, message, Tag, Space } from 'antd';
import { UploadOutlined, PlusOutlined, DeleteOutlined, EyeOutlined } from '@ant-design/icons';
import { motion } from 'framer-motion';
import styled from 'styled-components';
import { ResourceService } from '../utils/api';
import { useNavigate } from 'react-router-dom';
import { ResourceCategory } from '../types';

const UploadContainer = styled.div`
  min-height: 100vh;
  background: linear-gradient(135deg, #0a0a0a 0%, #1a1a2e 50%, #16213e 100%);
  padding: 20px;
`;

const UploadCard = styled(Card)`
  max-width: 800px;
  margin: 0 auto;
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;

  .ant-card-body {
    padding: 40px;
  }

  .ant-form-item-label > label {
    color: rgba(255, 255, 255, 0.8);
    font-weight: 500;
  }

  .ant-input, .ant-input-number, .ant-select-selector {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    color: white;
    
    &:hover, &:focus {
      border-color: #00d4ff;
      background: rgba(255, 255, 255, 0.15);
    }
  }

  .ant-upload {
    .ant-upload-drag {
      background: rgba(255, 255, 255, 0.05);
      border: 2px dashed rgba(255, 255, 255, 0.3);
      border-radius: 12px;
      
      &:hover {
        border-color: #00d4ff;
        background: rgba(0, 212, 255, 0.1);
      }
    }
    
    .ant-upload-drag-icon {
      color: #00d4ff;
    }
    
    .ant-upload-text {
      color: rgba(255, 255, 255, 0.8);
    }
    
    .ant-upload-hint {
      color: rgba(255, 255, 255, 0.5);
    }
  }

  .ant-steps {
    .ant-steps-item-finish .ant-steps-item-icon {
      background-color: #00d4ff;
      border-color: #00d4ff;
    }
    
    .ant-steps-item-active .ant-steps-item-icon {
      border-color: #00d4ff;
    }
    
    .ant-steps-item-title {
      color: rgba(255, 255, 255, 0.8) !important;
    }
    
    .ant-steps-item-description {
      color: rgba(255, 255, 255, 0.5) !important;
    }
  }
`;

const Title = styled.h1`
  text-align: center;
  color: white;
  font-size: 28px;
  font-weight: 700;
  margin-bottom: 30px;
  background: linear-gradient(135deg, #00d4ff, #ffffff);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
`;

const TagInput = styled.div`
  .tag-input {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 12px;
    
    .ant-tag {
      background: rgba(0, 212, 255, 0.2);
      border: 1px solid rgba(0, 212, 255, 0.5);
      color: #00d4ff;
      border-radius: 16px;
      padding: 4px 12px;
      display: flex;
      align-items: center;
      gap: 6px;
      
      .anticon {
        cursor: pointer;
        
        &:hover {
          color: #ff4d4f;
        }
      }
    }
  }
`;

const UploadResourcePage: React.FC = () => {
  const [form] = Form.useForm();
  const [currentStep, setCurrentStep] = useState(0);
  const [loading, setLoading] = useState(false);
  const [categories, setCategories] = useState<ResourceCategory[]>([]);
  const [tags, setTags] = useState<string[]>([]);
  const [newTag, setNewTag] = useState('');
  const [fileList, setFileList] = useState<any[]>([]);
  const [imageList, setImageList] = useState<any[]>([]);
  const navigate = useNavigate();

  useEffect(() => {
    loadCategories();
  }, []);

  const loadCategories = async () => {
    try {
      const response = await ResourceService.getCategories();
      if (response.success) {
        setCategories(response.data);
      }
    } catch (error) {
      message.error('加载分类失败');
    }
  };

  const handleAddTag = () => {
    if (newTag && !tags.includes(newTag)) {
      setTags([...tags, newTag]);
      setNewTag('');
    }
  };

  const handleRemoveTag = (tagToRemove: string) => {
    setTags(tags.filter(tag => tag !== tagToRemove));
  };

  const handleSubmit = async (values: any) => {
    setLoading(true);
    try {
      const resourceData = {
        ...values,
        tags,
        displayImages: imageList.map(img => img.response?.url || img.url)
      };
      
      const response = await ResourceService.uploadResource(resourceData);
      if (response.success) {
        // 上传文件
        if (fileList.length > 0) {
          await ResourceService.uploadResourceFile(response.data.id, fileList[0].originFileObj);
        }
        
        message.success('资源上传成功！');
        navigate('/user-center');
      } else {
        message.error(response.message || '上传失败');
      }
    } catch (error) {
      message.error('上传失败，请检查网络连接');
    } finally {
      setLoading(false);
    }
  };

  const steps = [
    {
      title: '基本信息',
      description: '填写资源基本信息'
    },
    {
      title: '详细描述',
      description: '添加详细描述和规格'
    },
    {
      title: '文件上传',
      description: '上传资源文件和图片'
    },
    {
      title: '预览提交',
      description: '预览并提交资源'
    }
  ];

  const renderStepContent = () => {
    switch (currentStep) {
      case 0:
        return (
          <>
            <Form.Item
              name="title"
              label="资源标题"
              rules={[{ required: true, message: '请输入资源标题' }]}
            >
              <Input placeholder="请输入资源标题" size="large" />
            </Form.Item>

            <Form.Item
              name="category"
              label="资源分类"
              rules={[{ required: true, message: '请选择资源分类' }]}
            >
              <Select placeholder="请选择资源分类" size="large">
                {categories.map(category => (
                  <Select.Option key={category.id} value={category.id}>
                    {category.name}
                  </Select.Option>
                ))}
              </Select>
            </Form.Item>

            <Form.Item
              name="price"
              label="资源价格"
              rules={[{ required: true, message: '请输入资源价格' }]}
            >
              <InputNumber
                placeholder="请输入价格"
                size="large"
                style={{ width: '100%' }}
                min={0}
                precision={2}
                addonBefore="¥"
              />
            </Form.Item>

            <Form.Item
              name="description"
              label="简短描述"
              rules={[{ required: true, message: '请输入资源描述' }]}
            >
              <Input.TextArea
                placeholder="请输入资源的简短描述"
                rows={4}
              />
            </Form.Item>

            <Form.Item label="标签">
              <TagInput>
                <div className="tag-input">
                  {tags.map(tag => (
                    <Tag key={tag}>
                      {tag}
                      <DeleteOutlined onClick={() => handleRemoveTag(tag)} />
                    </Tag>
                  ))}
                </div>
                <Input
                  placeholder="输入标签后按回车添加"
                  value={newTag}
                  onChange={(e) => setNewTag(e.target.value)}
                  onPressEnter={handleAddTag}
                  suffix={
                    <Button type="link" onClick={handleAddTag} disabled={!newTag}>
                      <PlusOutlined />
                    </Button>
                  }
                />
              </TagInput>
            </Form.Item>
          </>
        );

      case 1:
        return (
          <>
            <Form.Item
              name="detailDescription"
              label="详细描述"
            >
              <Input.TextArea
                placeholder="请输入资源的详细描述，包括功能特点、使用场景等"
                rows={6}
              />
            </Form.Item>

            <Form.Item
              name="usageGuide"
              label="使用指南"
            >
              <Input.TextArea
                placeholder="请输入使用指南，帮助用户更好地使用资源"
                rows={4}
              />
            </Form.Item>

            <Form.Item
              name="precautions"
              label="注意事项"
            >
              <Input.TextArea
                placeholder="请输入使用注意事项"
                rows={3}
              />
            </Form.Item>

            <Form.Item
              name="specifications"
              label="技术规格"
            >
              <Input.TextArea
                placeholder="请输入技术规格信息（JSON格式）"
                rows={4}
              />
            </Form.Item>
          </>
        );

      case 2:
        return (
          <>
            <Form.Item label="资源文件" required>
              <Upload.Dragger
                fileList={fileList}
                onChange={({ fileList }) => setFileList(fileList)}
                beforeUpload={() => false}
                maxCount={1}
              >
                <p className="ant-upload-drag-icon">
                  <UploadOutlined style={{ fontSize: '48px' }} />
                </p>
                <p className="ant-upload-text">点击或拖拽文件到此区域上传</p>
                <p className="ant-upload-hint">
                  支持单个文件上传，建议文件大小不超过100MB
                </p>
              </Upload.Dragger>
            </Form.Item>

            <Form.Item label="展示图片">
              <Upload
                listType="picture-card"
                fileList={imageList}
                onChange={({ fileList }) => setImageList(fileList)}
                beforeUpload={() => false}
                multiple
              >
                {imageList.length >= 8 ? null : (
                  <div>
                    <PlusOutlined />
                    <div style={{ marginTop: 8 }}>上传图片</div>
                  </div>
                )}
              </Upload>
            </Form.Item>
          </>
        );

      case 3:
        return (
          <div style={{ color: 'rgba(255, 255, 255, 0.8)' }}>
            <h3 style={{ color: '#00d4ff', marginBottom: '20px' }}>资源预览</h3>
            
            <div style={{ marginBottom: '16px' }}>
              <strong>标题：</strong>{form.getFieldValue('title')}
            </div>
            
            <div style={{ marginBottom: '16px' }}>
              <strong>分类：</strong>
              {categories.find(c => c.id === form.getFieldValue('category'))?.name}
            </div>
            
            <div style={{ marginBottom: '16px' }}>
              <strong>价格：</strong>¥{form.getFieldValue('price')}
            </div>
            
            <div style={{ marginBottom: '16px' }}>
              <strong>描述：</strong>{form.getFieldValue('description')}
            </div>
            
            <div style={{ marginBottom: '16px' }}>
              <strong>标签：</strong>
              <Space>
                {tags.map(tag => (
                  <Tag key={tag} color="blue">{tag}</Tag>
                ))}
              </Space>
            </div>
            
            <div style={{ marginBottom: '16px' }}>
              <strong>文件：</strong>
              {fileList.length > 0 ? fileList[0].name : '未上传'}
            </div>
            
            <div style={{ marginBottom: '16px' }}>
              <strong>图片：</strong>{imageList.length} 张
            </div>
          </div>
        );

      default:
        return null;
    }
  };

  return (
    <UploadContainer>
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6 }}
      >
        <UploadCard>
          <Title>上传资源</Title>
          
          <Steps current={currentStep} items={steps} style={{ marginBottom: '40px' }} />
          
          <Form
            form={form}
            layout="vertical"
            onFinish={handleSubmit}
            size="large"
          >
            {renderStepContent()}
            
            <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: '40px' }}>
              <Button
                disabled={currentStep === 0}
                onClick={() => setCurrentStep(currentStep - 1)}
              >
                上一步
              </Button>
              
              {currentStep < steps.length - 1 ? (
                <Button
                  type="primary"
                  onClick={() => setCurrentStep(currentStep + 1)}
                >
                  下一步
                </Button>
              ) : (
                <Button
                  type="primary"
                  htmlType="submit"
                  loading={loading}
                >
                  提交资源
                </Button>
              )}
            </div>
          </Form>
        </UploadCard>
      </motion.div>
    </UploadContainer>
  );
};

export default UploadResourcePage;